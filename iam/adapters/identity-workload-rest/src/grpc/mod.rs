//! gRPC delivery surface for workload-identity.
//! Implements both `WorkloadAuthorizer` and `WorkloadTokenValidator` tonic
//! service traits, delegating to the SAME `identity-workload-app`
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

use std::sync::Arc;

use iam_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;

use crate::{AuditSink, WorkloadAuthzState};

mod authorizer;
mod convert;
mod validator;

// Include tonic-generated stubs for iam.workload.v1.
pub mod proto {
    tonic::include_proto!("iam.workload.v1");
}

pub use proto::workload_authorizer_server::WorkloadAuthorizerServer;
pub use proto::workload_token_validator_server::WorkloadTokenValidatorServer;

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
