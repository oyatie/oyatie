//! `WorkloadAuthorizer` tonic service: the three authorize RPCs.

use tonic::{Request, Response, Status};

use iam_identity_workload_api::ClaimValueDto;
use iam_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository,
};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{Action, AuthorizationRequest};

use crate::{AuditEvent, AuditRecord, AuditSink, build_active_principal};

use super::WorkloadGrpcServer;
use super::convert::{
    authorize_outcome_label, best_effort_subject, best_effort_workload_id, decide_grpc,
    decode_authorize_with_token_request, outcome_to_proto_response, proto_context_to_domain,
    proto_resource_to_domain, run_authorize_with_token_grpc, verify_grpc_caller,
};
use super::proto::{
    AuthorizeRequest as ProtoAuthorizeRequest, AuthorizeResponse as ProtoAuthorizeResponse,
    AuthorizeWithTokenRequest as ProtoAuthorizeWithTokenRequest,
    BatchAuthorizeRequest as ProtoBatchAuthorizeRequest,
    BatchAuthorizeResponse as ProtoBatchAuthorizeResponse, DecisionEffect,
    workload_authorizer_server::WorkloadAuthorizer as WorkloadAuthorizerTrait,
};

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
