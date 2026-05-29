//! gRPC delivery surface for workload-identity — STUB (TDD red phase).
//!
//! This module declares the public types and tonic trait impls required by
//! the integration tests in `tests/grpc_authorize_deny.rs` so the test
//! binaries COMPILE but the impl bodies are `todo!()` so every test FAILS
//! at runtime.  The green phase (T2) replaces the `todo!()` bodies with the
//! real delegation logic.
//!
//! ## What the tests assert (acceptance criteria, T3)
//!
//! (a) Allowed principal -> DECISION_EFFECT_ALLOW.
//! (b) Forbidden principal -> DECISION_EFFECT_DENY response (NOT a tonic Err).
//! (c) Invalid token -> typed `ValidateTokenResponse` error; engine NOT consulted.
//! (d) Store/JWKS unavailable -> `Status::Unavailable`.
//!
//! ## Shared-core design (to be implemented in green phase)
//!
//! Both this module and the REST handlers (src/lib.rs) must delegate inward
//! to the same `oya-identity-workload-app` use-cases and OIDC adapter
//! primitives with no duplicated decision logic:
//!
//! - `AuthorizeWithToken` / `AuthorizeBatch` -> `authorize_with_token` app use-case.
//! - `Authorize` -> `build_active_principal` (crate fn, pub(crate) in lib.rs) +
//!   `authorizer_ref().authorize`.
//! - `ValidateToken` -> `validate_workload_token` (OIDC adapter).
//!
//! ## Fail-closed contract (to be enforced in green phase)
//!
//! - Authorization deny -> `AuthorizeResponse { effect: DECISION_EFFECT_DENY }` — never a tonic error.
//! - Token-validation failure -> `ValidateTokenResponse { ok: false, outcome: Error(...) }` — engine NOT consulted.
//! - Store / JWKS unavailable -> `tonic::Status::unavailable` for unary RPCs;
//!   per-item DENY decision value in batch.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use oya_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use oya_identity_workload_authz_cedar_adapter::WorkloadAuthorizer;

use crate::{AuditSink, WorkloadAuthzState};

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
    ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse,
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
/// Bodies are `todo!()` in this red-phase stub.
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
// WorkloadAuthorizer tonic impl — STUB
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
        _request: Request<ProtoAuthorizeWithTokenRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        let _ = &self.state;
        todo!("T2: implement AuthorizeWithToken delegating to authorize_with_token app use-case")
    }

    async fn authorize(
        &self,
        _request: Request<ProtoAuthorizeRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        let _ = &self.state;
        todo!("T2: implement Authorize delegating to build_active_principal + authorizer.authorize")
    }

    async fn authorize_batch(
        &self,
        _request: Request<ProtoBatchAuthorizeRequest>,
    ) -> Result<Response<ProtoBatchAuthorizeResponse>, Status> {
        let _ = &self.state;
        todo!("T2: implement AuthorizeBatch with per-item authorize_with_token + per-item audit")
    }
}

// =====================================================================
// WorkloadTokenValidator tonic impl — STUB
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
        _request: Request<ProtoValidateTokenRequest>,
    ) -> Result<Response<ProtoValidateTokenResponse>, Status> {
        let _ = &self.state;
        todo!("T2: implement ValidateToken delegating to validate_workload_token; token-fail = typed response not tonic Err")
    }
}
