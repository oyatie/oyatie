//! `WorkloadTokenValidator` tonic service: token introspection.

use tonic::{Request, Response, Status};

use iam_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_oidc::validate_workload_token;

use crate::{AuditEvent, AuditRecord, AuditSink};

use super::WorkloadGrpcServer;
use super::convert::{
    decide_grpc, oidc_error_to_kind, verify_grpc_caller, workload_state_to_proto,
};
use super::proto::{
    self, ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse, VerifiedPrincipal,
    workload_token_validator_server::WorkloadTokenValidator as WorkloadTokenValidatorTrait,
};

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
