//! gRPC decision surface (tonic).
//!
//! Serves `oya.cloud.iam.pdp.v1.CloudIamPdp` over the SAME
//! [`PdpState::decide`] core as REST, so the two protocols can never drift.
//!
//! A Deny is a DECISION response (`DECISION_EFFECT_DENY`), never an RPC
//! error. Refusals map to gRPC status errors and PEPs MUST treat any status
//! error as deny (fail-closed):
//!
//! | `PdpError`              | gRPC status           |
//! |-------------------------|-----------------------|
//! | `InvalidRequest`        | `INVALID_ARGUMENT`    |
//! | `UnknownAction`         | `INVALID_ARGUMENT`    |
//! | `Evaluation`            | `INVALID_ARGUMENT`    |
//! | `StalePolicyVersion`    | `FAILED_PRECONDITION` |
//! | `BundleRejected`        | `UNAVAILABLE`         |
//! | `DecisionIdUnavailable` | `INTERNAL`            |
//!
//! Proto→contract translation itself fails closed: a missing message field
//! or an unset attribute-value oneof is `INVALID_ARGUMENT`, never a silent
//! default.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic::{Request, Response, Status};

use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PdpError};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, PolicyVersion,
};

use crate::PdpState;

/// Generated protobuf/tonic bindings for `oya.cloud.iam.pdp.v1`.
pub mod proto {
    tonic::include_proto!("oya.cloud.iam.pdp.v1");
}

pub use proto::cloud_iam_pdp_server::CloudIamPdpServer;

/// The tonic service over the shared decision core.
pub struct CloudIamPdpService {
    state: Arc<PdpState>,
}

impl CloudIamPdpService {
    /// Build the service over the shared state.
    #[must_use]
    pub fn new(state: Arc<PdpState>) -> Self {
        Self { state }
    }
}

fn entity_ref_from_proto(
    entity_ref: Option<proto::EntityRef>,
    field: &str,
) -> Result<EntityRef, Status> {
    let entity_ref = entity_ref
        .ok_or_else(|| Status::invalid_argument(format!("{field} is required (fail-closed)")))?;
    Ok(EntityRef {
        entity_type: entity_ref.entity_type,
        entity_id: entity_ref.entity_id,
    })
}

fn json_value_from_proto(
    value: proto::AttributeValue,
    field: &str,
) -> Result<serde_json::Value, Status> {
    match value.value {
        Some(proto::attribute_value::Value::StringValue(s)) => Ok(serde_json::Value::String(s)),
        Some(proto::attribute_value::Value::BoolValue(b)) => Ok(serde_json::Value::Bool(b)),
        Some(proto::attribute_value::Value::LongValue(l)) => Ok(serde_json::json!(l)),
        None => Err(Status::invalid_argument(format!(
            "{field}: attribute value is unset (fail-closed; string/bool/long required)"
        ))),
    }
}

fn attribute_map_from_proto(
    attributes: std::collections::HashMap<String, proto::AttributeValue>,
    field: &str,
) -> Result<BTreeMap<String, serde_json::Value>, Status> {
    let mut out = BTreeMap::new();
    for (key, value) in attributes {
        let json = json_value_from_proto(value, &format!("{field}.{key}"))?;
        out.insert(key, json);
    }
    Ok(out)
}

fn entity_slice_from_proto(records: Vec<proto::EntityRecord>) -> Result<EntitySlice, Status> {
    let mut entities = Vec::with_capacity(records.len());
    for (index, record) in records.into_iter().enumerate() {
        let uid = entity_ref_from_proto(record.uid, &format!("entities[{index}].uid"))?;
        let attributes =
            attribute_map_from_proto(record.attributes, &format!("entities[{index}].attributes"))?;
        let parents = record
            .parents
            .into_iter()
            .map(|p| EntityRef {
                entity_type: p.entity_type,
                entity_id: p.entity_id,
            })
            .collect();
        entities.push(EntityRecord {
            uid,
            attributes,
            parents,
        });
    }
    Ok(EntitySlice { entities })
}

// SVID caller-auth seam (G002 slice-1; ADR-0561): the mTLS PEP
// (`crate::mtls::SpiffeCallerAuth::authenticate_caller`) runs BEFORE this
// translation, deriving the authorized tenant from the verified peer SVID and
// passing it in place of `request.tenant_id`. The live wiring binds the peer
// leaf from the rustls handshake on `server.rs`'s listeners — the DEFERRED
// slice-1b transport (ADR-0561 D5). Until that lands, `request.tenant_id` is
// still threaded here; the PEP + its 6 RED fixtures (`crate::mtls::tests`) are
// the in-process closure that proves the bind-or-deny logic, and the bound
// tenant supersedes the body tenant at the call site, never inside this
// pure proto→contract translation.
fn contract_request_from_proto(
    request: proto::AuthorizeRequest,
) -> Result<(AuthorizationRequest, EntitySlice), Status> {
    let principal = entity_ref_from_proto(request.principal, "principal")?;
    let resource = entity_ref_from_proto(request.resource, "resource")?;
    let context = attribute_map_from_proto(request.context, "context")?;
    let min_policy_version = if request.min_policy_version.is_empty() {
        None
    } else {
        Some(
            PolicyVersion::new(request.min_policy_version).map_err(|violations| {
                Status::invalid_argument(format!("min_policy_version rejected: {violations:?}"))
            })?,
        )
    };
    let entities = entity_slice_from_proto(request.entities)?;
    Ok((
        AuthorizationRequest {
            request_id: request.request_id,
            tenant_id: request.tenant_id,
            principal,
            action: request.action,
            resource,
            context,
            min_policy_version,
        },
        entities,
    ))
}

fn proto_response_from_contract(response: AuthorizationResponse) -> proto::AuthorizeResponse {
    let decision = match response.decision {
        Decision::Allow => proto::DecisionEffect::Allow,
        Decision::Deny => proto::DecisionEffect::Deny,
    };
    proto::AuthorizeResponse {
        decision_id: response.decision_id,
        request_id: response.request_id,
        decision: decision.into(),
        policy_version: response.policy_version.as_str().to_owned(),
        determining_policy_ids: response.determining_policy_ids,
        obligations: response
            .obligations
            .into_iter()
            .map(|o| proto::Obligation {
                obligation_id: o.obligation_id,
                parameters: o.parameters.into_iter().collect(),
            })
            .collect(),
    }
}

fn status_from_refusal(error: &PdpError) -> Status {
    match error {
        PdpError::InvalidRequest(_)
        | PdpError::UnknownAction { .. }
        | PdpError::Evaluation { .. } => Status::invalid_argument(error.to_string()),
        PdpError::StalePolicyVersion { .. } => Status::failed_precondition(error.to_string()),
        PdpError::BundleRejected { .. } => Status::unavailable(error.to_string()),
        PdpError::DecisionIdUnavailable { .. } => Status::internal(error.to_string()),
    }
}

#[tonic::async_trait]
impl proto::cloud_iam_pdp_server::CloudIamPdp for CloudIamPdpService {
    async fn authorize(
        &self,
        request: Request<proto::AuthorizeRequest>,
    ) -> Result<Response<proto::AuthorizeResponse>, Status> {
        let (contract_request, entities) = contract_request_from_proto(request.into_inner())?;
        match self.state.decide(&contract_request, &entities) {
            Ok(response) => Ok(Response::new(proto_response_from_contract(response))),
            Err(error) => {
                PdpState::log_refusal(&contract_request.request_id, &error);
                Err(status_from_refusal(&error))
            }
        }
    }

    async fn get_loaded_policy_version(
        &self,
        _request: Request<proto::GetLoadedPolicyVersionRequest>,
    ) -> Result<Response<proto::GetLoadedPolicyVersionResponse>, Status> {
        Ok(Response::new(proto::GetLoadedPolicyVersionResponse {
            policy_version: self.state.loaded_policy_version().as_str().to_owned(),
        }))
    }
}

/// Serve the gRPC surface on `incoming` until `shutdown` resolves
/// (graceful drain).
///
/// # Errors
/// Returns the tonic transport error when serving fails.
pub async fn serve<F>(
    state: Arc<PdpState>,
    incoming: TcpIncoming,
    shutdown: F,
) -> Result<(), tonic::transport::Error>
where
    F: Future<Output = ()> + Send + 'static,
{
    Server::builder()
        .add_service(CloudIamPdpServer::new(CloudIamPdpService::new(state)))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await
}
