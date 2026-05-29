//! gRPC delivery surface for workload-identity (ADR-0509 single-crate extension).
//!
//! Implements `WorkloadAuthorizer` and `WorkloadTokenValidator` from
//! `microservices/identity/contracts/proto/workload.proto`
//! (package `oya.identity.workload.v1`).
//!
//! ## Shared-core design
//!
//! Both this module and the REST handlers (src/lib.rs) delegate inward to the
//! same `oya-identity-workload-app` use-cases and OIDC adapter primitives.
//! There is no duplicated decision logic:
//!
//! - `AuthorizeWithToken` / `AuthorizeBatch` -> `authorize_with_token` app use-case.
//! - `Authorize` -> `build_active_principal` (crate-private REST helper) +
//!   `authorizer_ref().authorize`. This mirrors the REST `/authorize` handler exactly;
//!   the shared core for this RPC is the crate's own `build_active_principal` fn.
//! - `ValidateToken` -> `validate_workload_token` (OIDC adapter).
//!
//! ## Fail-closed contract (proto header + PRD §3.4/§3.5/§5)
//!
//! - Authorization deny -> `AuthorizeResponse { effect: DECISION_EFFECT_DENY }` — never a tonic error.
//! - Token-validation failure -> `ValidateTokenResponse { ok: false, outcome: Error(...) }` — engine NOT consulted.
//! - Store / JWKS unavailable -> `tonic::Status::unavailable` for unary RPCs;
//!   per-item DENY decision value in batch (preserving REST batch parity).
//!
//! ## Audit emission (AC-W-13)
//!
//! One `AuditRecord` is emitted per authorize call and per token-validation,
//! identical to the REST path.

use std::collections::BTreeMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use oya_identity_workload_api::{AuthorizeRequest as ApiAuthorizeRequest, ClaimValueDto, ResourceDto};
use oya_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token,
};
use oya_identity_workload_authz_cedar_adapter::WorkloadAuthorizer;
use oya_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue as DomainClaimValue,
    DecisionReason, WorkloadState as DomainWorkloadState,
};
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
    DecisionReason as ProtoDecisionReason,
    DecisionReasonKind,
    ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse,
    ValidationError as ProtoValidationError,
    ValidationErrorKind,
    VerifiedPrincipal as ProtoVerifiedPrincipal,
    WorkloadState as ProtoWorkloadState,
    claim_value,
    validate_token_response,
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
    /// Full hot path: validate JWT -> resolve principal -> authorize.
    /// Mirrors `authorize_with_token_handler` in src/lib.rs.
    async fn authorize_with_token(
        &self,
        request: Request<ProtoAuthorizeWithTokenRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        let req = request.into_inner();
        let api_req = oya_identity_workload_api::AuthorizeWithTokenRequest {
            token: req.token,
            action: req.action,
            resource: proto_resource_to_dto(req.resource),
            context: proto_context_to_btree(req.context),
        };
        let outcome = grpc_run_authorize_with_token(&self.state, &api_req);
        let workload_id = grpc_workload_id_from_token(&self.state, &api_req.token);
        outcome_to_grpc_response(self.state.audit(), workload_id, &outcome)
    }

    /// Authorize an already-verified principal supplied by a trusted PEP.
    /// Mirrors `authorize_handler` in src/lib.rs.
    /// Uses `build_active_principal` (crate fn) as the shared core for this path.
    async fn authorize(
        &self,
        request: Request<ProtoAuthorizeRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        let req = request.into_inner();
        let api_req = ApiAuthorizeRequest {
            tenant_id: req.tenant_id,
            workload_id: req.workload_id,
            owning_capability: req.owning_capability,
            scopes: req.scopes,
            claims: proto_context_to_btree(req.claims),
            action: req.action,
            resource: proto_resource_to_dto(req.resource),
            context: proto_context_to_btree(req.context),
        };
        let principal = match build_active_principal(&api_req) {
            Ok(p) => p,
            Err(envelope) => {
                return Err(Status::invalid_argument(format!("{envelope:?}")));
            }
        };
        let workload_id = principal.workload_id().as_str().to_owned();
        let mut authz_request = AuthorizationRequest::new(
            principal,
            Action::new(api_req.action.clone()),
            api_req.resource.clone().into_domain(),
        );
        authz_request.context = api_req
            .context
            .iter()
            .map(|(k, v)| (k.clone(), DomainClaimValue::from(v.clone())))
            .collect();
        let decision = self.state.authorizer_ref().authorize(&authz_request);
        let outcome = AuthorizeOutcome::Decided(decision);
        outcome_to_grpc_response(self.state.audit(), Some(workload_id), &outcome)
    }

    /// Batch authorize — one decision per item in order.
    /// Mirrors `authorize_batch_handler` in src/lib.rs.
    /// Store/JWKS outage on a single item -> DENY decision VALUE (not top-level Unavailable).
    async fn authorize_batch(
        &self,
        request: Request<ProtoBatchAuthorizeRequest>,
    ) -> Result<Response<ProtoBatchAuthorizeResponse>, Status> {
        let req = request.into_inner();
        let mut decisions = Vec::with_capacity(req.requests.len());
        for item in &req.requests {
            let api_req = oya_identity_workload_api::AuthorizeWithTokenRequest {
                token: item.token.clone(),
                action: item.action.clone(),
                resource: proto_resource_to_dto(item.resource.clone()),
                context: proto_context_to_btree(item.context.clone()),
            };
            let outcome = grpc_run_authorize_with_token(&self.state, &api_req);
            let workload_id = grpc_workload_id_from_token(&self.state, &api_req.token);
            let label = batch_outcome_label(&outcome);
            self.state.audit().record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                label,
                None,
            ));
            decisions.push(outcome_to_authorize_response(&outcome.decision()));
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
    /// Validate a workload JWT, return the projected principal identity.
    /// Token-validation failure -> typed `ValidateTokenResponse` error (engine NOT consulted).
    /// JWKS unavailable -> `Status::unavailable`.
    async fn validate_token(
        &self,
        request: Request<ProtoValidateTokenRequest>,
    ) -> Result<Response<ProtoValidateTokenResponse>, Status> {
        let req = request.into_inner();
        let now = (self.state.now_provider_ref())();
        match validate_workload_token(
            &req.token,
            self.state.jwks_ref(),
            self.state.config_ref(),
            now,
        ) {
            Ok(principal) => {
                self.state.audit().record(AuditRecord::new(
                    AuditEvent::TokenValidation,
                    Some(principal.workload_id().as_str().to_owned()),
                    "validated",
                    None,
                ));
                let verified = ProtoVerifiedPrincipal {
                    tenant_id: principal.tenant_id().as_str().to_owned(),
                    workload_id: principal.workload_id().as_str().to_owned(),
                    owning_capability: principal.owning_capability().as_str().to_owned(),
                    trust_domain: principal.trust_domain().as_str().to_owned(),
                    state: proto_workload_state(principal.state()) as i32,
                    scopes: principal.scopes().to_vec(),
                };
                Ok(Response::new(ProtoValidateTokenResponse {
                    ok: true,
                    outcome: Some(validate_token_response::Outcome::Principal(verified)),
                }))
            }
            Err(error) => {
                self.state.audit().record(AuditRecord::new(
                    AuditEvent::TokenValidation,
                    None,
                    "validation-failed",
                    Some(error.to_string()),
                ));
                let (kind, detail) = oidc_error_to_proto(&error);
                Ok(Response::new(ProtoValidateTokenResponse {
                    ok: false,
                    outcome: Some(validate_token_response::Outcome::Error(
                        ProtoValidationError {
                            kind: kind as i32,
                            detail,
                        },
                    )),
                }))
            }
        }
    }
}

// =====================================================================
// Outcome -> gRPC response helpers
// =====================================================================

/// Map an `AuthorizeOutcome` to a tonic `Result<Response<AuthorizeResponse>>`.
///
/// Deny -> `DECISION_EFFECT_DENY` response value (never a tonic error).
/// `StoreUnavailable` -> `Status::unavailable`.
/// `TokenRejected` -> `DECISION_EFFECT_DENY` response (typed, not a transport error).
fn outcome_to_grpc_response<S: AuditSink>(
    audit: &S,
    workload_id: Option<String>,
    outcome: &AuthorizeOutcome,
) -> Result<Response<ProtoAuthorizeResponse>, Status> {
    match outcome {
        AuthorizeOutcome::StoreUnavailable => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "store-unavailable",
                None,
            ));
            Err(Status::unavailable("store or JWKS unavailable"))
        }
        AuthorizeOutcome::TokenRejected => {
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                "token-rejected",
                None,
            ));
            Ok(Response::new(deny_response(DecisionReasonKind::DefaultDeny, None)))
        }
        _ => {
            let decision = outcome.decision();
            let (label, detail) = audit_label_and_detail(outcome, &decision);
            audit.record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                label,
                detail,
            ));
            Ok(Response::new(outcome_to_authorize_response(&decision)))
        }
    }
}

/// Build a DENY `AuthorizeResponse` with the given reason.
fn deny_response(kind: DecisionReasonKind, policy_id: Option<String>) -> ProtoAuthorizeResponse {
    ProtoAuthorizeResponse {
        effect: DecisionEffect::Deny as i32,
        reason: Some(ProtoDecisionReason {
            kind: kind as i32,
            policy_id,
            state: None,
        }),
    }
}

/// Convert an `AuthorizationDecision` to a proto `AuthorizeResponse`.
fn outcome_to_authorize_response(decision: &AuthorizationDecision) -> ProtoAuthorizeResponse {
    let effect = if decision.is_allow() {
        DecisionEffect::Allow
    } else {
        DecisionEffect::Deny
    };
    ProtoAuthorizeResponse {
        effect: effect as i32,
        reason: Some(decision_reason_proto(decision.reason())),
    }
}

/// Build proto `DecisionReason` from domain `DecisionReason`.
fn decision_reason_proto(reason: &DecisionReason) -> ProtoDecisionReason {
    match reason {
        DecisionReason::ExplicitPermit { policy_id } => ProtoDecisionReason {
            kind: DecisionReasonKind::ExplicitPermit as i32,
            policy_id: Some(policy_id.clone()),
            state: None,
        },
        DecisionReason::ExplicitForbid { policy_id } => ProtoDecisionReason {
            kind: DecisionReasonKind::ExplicitForbid as i32,
            policy_id: Some(policy_id.clone()),
            state: None,
        },
        DecisionReason::DefaultDeny => ProtoDecisionReason {
            kind: DecisionReasonKind::DefaultDeny as i32,
            policy_id: None,
            state: None,
        },
        DecisionReason::PrincipalNotOperational { state } => ProtoDecisionReason {
            kind: DecisionReasonKind::PrincipalNotOperational as i32,
            policy_id: None,
            state: Some(domain_state_label(*state).to_owned()),
        },
    }
}

/// Return `(audit_label, detail)` for an outcome/decision pair.
fn audit_label_and_detail(
    outcome: &AuthorizeOutcome,
    decision: &AuthorizationDecision,
) -> (&'static str, Option<String>) {
    match outcome {
        AuthorizeOutcome::PrincipalUnknown => ("deny", Some("principal-unknown".to_owned())),
        AuthorizeOutcome::Revoked => ("deny", Some("revoked".to_owned())),
        AuthorizeOutcome::Decided(d) if d.is_allow() => ("allow", decision_detail(decision)),
        AuthorizeOutcome::Decided(_) => ("deny", decision_detail(decision)),
        // StoreUnavailable and TokenRejected handled above.
        _ => ("deny", None),
    }
}

fn decision_detail(decision: &AuthorizationDecision) -> Option<String> {
    match decision.reason() {
        DecisionReason::ExplicitPermit { policy_id }
        | DecisionReason::ExplicitForbid { policy_id } => Some(policy_id.clone()),
        DecisionReason::DefaultDeny => Some("default-deny".to_owned()),
        DecisionReason::PrincipalNotOperational { .. } => Some("not-operational".to_owned()),
    }
}

/// Outcome label for batch-item audit records.
fn batch_outcome_label(outcome: &AuthorizeOutcome) -> &'static str {
    match outcome {
        AuthorizeOutcome::Decided(d) if d.is_allow() => "allow",
        AuthorizeOutcome::Decided(_) => "deny",
        AuthorizeOutcome::TokenRejected => "token-rejected",
        AuthorizeOutcome::PrincipalUnknown => "principal-unknown",
        AuthorizeOutcome::Revoked => "revoked",
        AuthorizeOutcome::StoreUnavailable => "store-unavailable",
    }
}

// =====================================================================
// OidcValidationError -> proto mapping
// =====================================================================

/// Map `OidcValidationError` to a `(ValidationErrorKind, detail)` pair.
///
/// OidcValidationError has ~16 variants; proto has 12 ValidationErrorKind values.
/// MalformedKey has no exact counterpart and collapses to Malformed.
/// MissingClaim and Domain collapse to MissingClaim (closest semantic fit).
fn oidc_error_to_proto(error: &OidcValidationError) -> (ValidationErrorKind, String) {
    let kind = match error {
        OidcValidationError::MalformedToken
        | OidcValidationError::DecodeError
        | OidcValidationError::MalformedKey => ValidationErrorKind::Malformed,
        OidcValidationError::AlgNone => ValidationErrorKind::AlgNone,
        OidcValidationError::InvalidType => ValidationErrorKind::InvalidType,
        OidcValidationError::UntrustedKeySourceUrl => ValidationErrorKind::UntrustedKeySourceUrl,
        OidcValidationError::AlgorithmMismatch | OidcValidationError::UnsupportedAlgorithm => {
            ValidationErrorKind::AlgorithmMismatch
        }
        OidcValidationError::UnknownKey => ValidationErrorKind::UnknownKey,
        OidcValidationError::SignatureInvalid => ValidationErrorKind::SignatureInvalid,
        OidcValidationError::IssuerMismatch => ValidationErrorKind::IssuerMismatch,
        OidcValidationError::AudienceMismatch => ValidationErrorKind::AudienceMismatch,
        OidcValidationError::Expired => ValidationErrorKind::Expired,
        OidcValidationError::NotYetValid => ValidationErrorKind::NotYetValid,
        OidcValidationError::MissingClaim(_) | OidcValidationError::Domain(_) => {
            ValidationErrorKind::MissingClaim
        }
    };
    (kind, error.to_string())
}

// =====================================================================
// Domain type conversions
// =====================================================================

/// Convert domain `WorkloadState` to the lowercase wire label (mirrors `state_label` in src/lib.rs).
fn domain_state_label(state: DomainWorkloadState) -> &'static str {
    match state {
        DomainWorkloadState::Provisioned => "provisioned",
        DomainWorkloadState::Active => "active",
        DomainWorkloadState::Suspended => "suspended",
        DomainWorkloadState::Retired => "retired",
    }
}

/// Map domain `WorkloadState` to proto `WorkloadState`.
fn proto_workload_state(state: DomainWorkloadState) -> ProtoWorkloadState {
    match state {
        DomainWorkloadState::Provisioned => ProtoWorkloadState::Provisioned,
        DomainWorkloadState::Active => ProtoWorkloadState::Active,
        DomainWorkloadState::Suspended => ProtoWorkloadState::Suspended,
        DomainWorkloadState::Retired => ProtoWorkloadState::Retired,
    }
}

/// Convert a proto `Resource` option to an API `ResourceDto`.
/// An absent resource is an empty type+id (safe default for the PEP).
fn proto_resource_to_dto(resource: Option<proto::Resource>) -> ResourceDto {
    resource
        .map(|r| ResourceDto {
            resource_type: r.resource_type,
            resource_id: r.resource_id,
        })
        .unwrap_or_else(|| ResourceDto {
            resource_type: String::new(),
            resource_id: String::new(),
        })
}

/// Convert a proto context map to a `BTreeMap<String, ClaimValueDto>`.
fn proto_context_to_btree(
    context: std::collections::HashMap<String, proto::ClaimValue>,
) -> BTreeMap<String, ClaimValueDto> {
    context
        .into_iter()
        .filter_map(|(k, v)| proto_claim_to_dto(v).map(|dto| (k, dto)))
        .collect()
}

/// Convert a single proto `ClaimValue` to an API `ClaimValueDto`.
fn proto_claim_to_dto(v: proto::ClaimValue) -> Option<ClaimValueDto> {
    use claim_value::Value;
    match v.value? {
        Value::Text(s) => Some(ClaimValueDto::Text(s)),
        Value::Boolean(b) => Some(ClaimValueDto::Bool(b)),
        Value::Integer(i) => Some(ClaimValueDto::Int(i)),
        Value::TextList(l) => Some(ClaimValueDto::TextList(l.values)),
    }
}

// =====================================================================
// State delegate helpers (call into WorkloadAuthzState internals via
// the pub(crate) accessors defined in src/lib.rs)
// =====================================================================

/// Run the token-bearing authorize use-case against the mutex-guarded state.
fn grpc_run_authorize_with_token<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    request: &oya_identity_workload_api::AuthorizeWithTokenRequest,
) -> AuthorizeOutcome
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
    authorize_with_token(
        &*repo_guard,
        &*denylist_guard,
        state.authorizer_ref(),
        state.jwks_ref(),
        state.config_ref(),
        now,
        &request.token,
        request.action(),
        request.resource.clone().into_domain(),
        request.context_domain(),
    )
}

/// Best-effort workload-id extraction from a token (for audit subject field).
fn grpc_workload_id_from_token<R, D, A, S>(
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
