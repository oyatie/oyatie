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
//! | `AuditChainEmission`    | `INTERNAL`            |
//! | `RuntimeTimeout`        | `DEADLINE_EXCEEDED`   |
//! | `RuntimePanic`          | `INTERNAL`            |
//! | `CircuitOpen`           | `UNAVAILABLE`         |
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

use os_trustd_domain::TrustBundle;
use os_trustd_domain::signer::EcdsaP256Signer;
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PdpError};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, PolicyVersion,
};

use crate::PdpState;
use crate::mtls::SpiffeCallerAuth;
use crate::mtls_transport::PeerCertInfo;

/// Generated protobuf/tonic bindings for `oya.cloud.iam.pdp.v1`.
pub mod proto {
    tonic::include_proto!("oya.cloud.iam.pdp.v1");
}

pub use proto::cloud_iam_pdp_server::CloudIamPdpServer;

/// The tonic service over the shared decision core.
///
/// When `bundle` is `Some`, the mTLS PEP is enforced: BEFORE the proto→contract
/// translation, the verified peer SVID (captured by the rustls handshake into a
/// [`PeerCertInfo`] request extension) is authenticated and its tenant binds the
/// decision, superseding the request-body `tenant_id` (the #717 closure). When
/// `bundle` is `None` (plain-TCP boot), the legacy verbatim-tenant path runs.
pub struct CloudIamPdpService {
    state: Arc<PdpState>,
    bundle: Option<Arc<TrustBundle<EcdsaP256Signer>>>,
    /// The PDP's own cell authority (`oyatie.cell-<id>`), used to pin a caller's
    /// cell. `None` ⇒ no cell pin (legacy / server leaf without a SPIFFE id).
    expected_cell_authority: Option<String>,
}

impl CloudIamPdpService {
    /// Build the service over the shared state with NO caller authentication
    /// (plain-TCP boot path).
    #[must_use]
    pub fn new(state: Arc<PdpState>) -> Self {
        Self {
            state,
            bundle: None,
            expected_cell_authority: None,
        }
    }

    /// Build the service with the mTLS PEP enforced over `bundle`, pinning the
    /// caller's cell to `expected_cell_authority` when supplied (the PDP's own
    /// cell, derived from its server SVID identity).
    #[must_use]
    pub fn with_caller_auth(
        state: Arc<PdpState>,
        bundle: Arc<TrustBundle<EcdsaP256Signer>>,
        expected_cell_authority: Option<String>,
    ) -> Self {
        Self {
            state,
            bundle: Some(bundle),
            expected_cell_authority,
        }
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
        PdpError::AuditChainEmission { .. } => Status::internal(error.to_string()),
        PdpError::RuntimeTimeout { .. } => Status::deadline_exceeded(error.to_string()),
        PdpError::RuntimePanic { .. } => Status::internal(error.to_string()),
        PdpError::CircuitOpen { .. } => Status::unavailable(error.to_string()),
    }
}

#[tonic::async_trait]
impl proto::cloud_iam_pdp_server::CloudIamPdp for CloudIamPdpService {
    async fn authorize(
        &self,
        request: Request<proto::AuthorizeRequest>,
    ) -> Result<Response<proto::AuthorizeResponse>, Status> {
        // mTLS PEP (the #717 closure): authenticate the caller by its verified
        // peer SVID and bind the tenant from the SVID path BEFORE translating
        // the body. The body `tenant_id` is only a cross-check input; any
        // mismatch is PermissionDenied (never INVALID_ARGUMENT, never a
        // verbatim-trust fall-through).
        let svid_tenant = if let Some(bundle) = &self.bundle {
            let peer_leaf = request
                .extensions()
                .get::<PeerCertInfo>()
                .and_then(|info| info.leaf_der.as_deref());
            let pep = match &self.expected_cell_authority {
                Some(cell) => SpiffeCallerAuth::with_cell_pin(bundle, cell.as_str()),
                None => SpiffeCallerAuth::new(bundle),
            }
            .map_err(|err| {
                // A serving process always has a non-empty bundle (boot-refuses
                // otherwise); treat any deviation as fail-closed deny.
                Status::new(tonic::Code::PermissionDenied, err.to_string())
            })?;
            let requested = request.get_ref().tenant_id.clone();
            let bound = pep
                .authenticate_caller(peer_leaf, &requested, now_secs())
                .map_err(|rej| rej.to_grpc_status())?;
            Some(bound.as_str().to_owned())
        } else {
            None
        };

        let (mut contract_request, entities) = contract_request_from_proto(request.into_inner())?;
        // Replace the verbatim body tenant with the SVID-derived tenant.
        if let Some(tenant) = svid_tenant {
            contract_request.tenant_id = tenant;
        }
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

/// Current wall-clock as epoch seconds for SVID validity checks. A clock before
/// the epoch yields `0` (every short-TTL SVID is then expired ⇒ fail-closed
/// DENY, never a spurious accept).
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Serve the gRPC surface on `incoming` (plain TCP) until `shutdown` resolves
/// (graceful drain). No caller authentication — the legacy verbatim-tenant path.
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

/// Serve the gRPC surface over mTLS: each connection terminates a rustls
/// handshake requiring a verified client SVID, and the PEP binds the caller's
/// tenant. `incoming` yields TLS-terminated streams carrying the captured peer
/// leaf (failed handshakes are already dropped upstream).
///
/// # Errors
/// Returns the tonic transport error when serving fails.
pub async fn serve_mtls<F, I>(
    state: Arc<PdpState>,
    bundle: Arc<TrustBundle<EcdsaP256Signer>>,
    expected_cell_authority: Option<String>,
    incoming: I,
    shutdown: F,
) -> Result<(), tonic::transport::Error>
where
    F: Future<Output = ()> + Send + 'static,
    I: futures_core::Stream<
            Item = Result<crate::mtls_transport::PeerCertTlsStream, std::io::Error>,
        > + Send
        + 'static,
{
    Server::builder()
        .add_service(CloudIamPdpServer::new(
            CloudIamPdpService::with_caller_auth(state, bundle, expected_cell_authority),
        ))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await
}

#[cfg(test)]
mod tests {
    use tonic::Code;

    use super::*;

    #[test]
    fn runtime_refusals_have_fail_closed_grpc_mappings() {
        assert_eq!(
            status_from_refusal(&PdpError::RuntimeTimeout { deadline_ms: 25 }).code(),
            Code::DeadlineExceeded
        );
        assert_eq!(
            status_from_refusal(&PdpError::RuntimePanic {
                detail: "panic".to_owned(),
            })
            .code(),
            Code::Internal
        );
        assert_eq!(
            status_from_refusal(&PdpError::AuditChainEmission {
                detail: "append failed".to_owned(),
            })
            .code(),
            Code::Internal
        );
        assert_eq!(
            status_from_refusal(&PdpError::CircuitOpen {
                consecutive_failures: 3,
            })
            .code(),
            Code::Unavailable
        );
    }
}
