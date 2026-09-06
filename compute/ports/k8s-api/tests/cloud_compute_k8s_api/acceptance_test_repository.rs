use std::collections::BTreeMap;
use std::sync::Mutex;

use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CloudComputeK8sAcceptCreateIntentCommand,
    CloudComputeK8sAcceptanceContract, CloudComputeK8sAcceptanceRepository,
    CloudComputeK8sAcceptanceRepositoryError, CloudComputeK8sAcceptedCreateIntent,
    CloudComputeK8sApiAuthorization, CloudComputeK8sApiAuthorizationProof,
    CloudComputeK8sApiBoundaryContext, CloudComputeK8sApiPrincipal,
    CloudComputeK8sClusterCreateIntent, CloudComputeK8sCreateAcceptanceApiRequest,
    CloudComputeK8sNodePoolFlavorSpec, CloudComputeK8sNodePoolIntent, CloudComputeK8sOperationKey,
    CloudComputeK8sOperationLookup, CloudComputeK8sOperationReadApiRequest,
    CloudComputeK8sOperationSnapshot, CloudComputeK8sReadCreateOperationQuery,
    CloudComputeK8sRepositoryFuture, CloudComputeK8sTrustedAuthorizationVerifier,
};
use shared_resource_provider_contract_kernel::OperationState;

pub const CLUSTER_ID: &str = "oyatie:cloud:region-home:ten_alpha:k8s:prod";
pub const ACCEPTED_AT: u64 = 1_700_100_010;
const EVALUATED_AT: u64 = 1_700_099_500;

pub fn pending_intent() -> CloudComputeK8sClusterCreateIntent {
    CloudComputeK8sClusterCreateIntent {
        resource_id: CLUSTER_ID.to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        flavor: "high_availability".to_string(),
        control_plane_version: "v1.30.2-oyatie.1".to_string(),
        control_plane_private: true,
        node_pools: ["a", "b", "c"].into_iter().map(node_pool).collect(),
        residency: "strict_home_region".to_string(),
        data_class: "PUBLIC".to_string(),
    }
}

fn node_pool(suffix: &str) -> CloudComputeK8sNodePoolIntent {
    let az = format!("region-home-{suffix}");
    CloudComputeK8sNodePoolIntent {
        id: format!("np_{suffix}"),
        az: az.clone(),
        cell_id: format!("cell-{az}-001"),
        subnet_id: format!("oyatie:cloud:region-home:ten_alpha:subnet:prod-{suffix}"),
        security_groups: vec![format!("sg_np_{suffix}_web"), format!("sg_np_{suffix}_app")],
        flavor: CloudComputeK8sNodePoolFlavorSpec {
            class: "general_purpose".to_string(),
            vcpu: 4,
            memory_gb: 16,
            gpu_count: 0,
            local_ssd_gb: 100,
        },
        min_nodes: 1,
        max_nodes: 5,
        autoscaling_enabled: true,
    }
}

pub fn pending_request(request_id: &str, key: &str) -> CloudComputeK8sCreateAcceptanceApiRequest {
    CloudComputeK8sCreateAcceptanceApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: boundary(request_id, key),
        principal: principal(),
        authorization: authorization("create-decision"),
        body: pending_intent(),
    }
}

pub fn read_request(request_id: &str, key: &str) -> CloudComputeK8sOperationReadApiRequest {
    CloudComputeK8sOperationReadApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: boundary(request_id, key),
        principal: principal(),
        authorization: authorization("read-decision"),
    }
}

fn boundary(request_id: &str, key: &str) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: key.to_string(),
    }
}

fn principal() -> CloudComputeK8sApiPrincipal {
    CloudComputeK8sApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: "sp_compute".to_string(),
    }
}

fn authorization(decision_id: &str) -> CloudComputeK8sApiAuthorization {
    CloudComputeK8sApiAuthorization {
        tenant_id: "ignored-caller-tenant".to_string(),
        principal_id: "ignored-caller-principal".to_string(),
        decision_id: decision_id.to_string(),
        allowed_surfaces: Vec::new(),
        proof: None,
    }
}

pub fn verifier(decision_id: &str, surface: &str) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(EVALUATED_AT).with_authorization_proof(
        CloudComputeK8sApiAuthorizationProof {
            tenant_id: "ten_alpha".to_string(),
            principal_id: "sp_compute".to_string(),
            surface: surface.to_string(),
            decision_id: decision_id.to_string(),
            verified: true,
            issued_at_epoch_seconds: 1_700_099_000,
            expires_at_epoch_seconds: 1_700_100_000,
        },
    )
}

pub fn pending_create_verifier(
    request: &CloudComputeK8sCreateAcceptanceApiRequest,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    verifier(
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    )
}

#[derive(Default)]
pub struct AcceptanceTestRepository {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    operations: BTreeMap<CloudComputeK8sOperationKey, CloudComputeK8sOperationSnapshot>,
    last_command: Option<CloudComputeK8sAcceptCreateIntentCommand>,
    accept_error: Option<CloudComputeK8sAcceptanceRepositoryError>,
    read_error: Option<CloudComputeK8sAcceptanceRepositoryError>,
    accept_calls: usize,
    read_calls: usize,
}

impl AcceptanceTestRepository {
    pub fn calls(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.accept_calls, state.read_calls)
    }

    pub fn last_command(&self) -> Option<CloudComputeK8sAcceptCreateIntentCommand> {
        self.state.lock().unwrap().last_command.clone()
    }

    pub fn fail_accept_with(&self, error: CloudComputeK8sAcceptanceRepositoryError) {
        self.state.lock().unwrap().accept_error = Some(error);
    }

    pub fn fail_read_with(&self, error: CloudComputeK8sAcceptanceRepositoryError) {
        self.state.lock().unwrap().read_error = Some(error);
    }

    pub fn mutate_snapshot(&self, mutate: impl FnOnce(&mut CloudComputeK8sOperationSnapshot)) {
        let mut state = self.state.lock().unwrap();
        mutate(state.operations.values_mut().next().unwrap());
    }
}

impl CloudComputeK8sAcceptanceRepository for AcceptanceTestRepository {
    fn accept_create_intent<'a>(
        &'a self,
        command: CloudComputeK8sAcceptCreateIntentCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sOperationSnapshot, CloudComputeK8sAcceptanceRepositoryError>,
    > {
        Box::pin(async move {
            let mut state = self.state.lock().unwrap();
            state.accept_calls += 1;
            state.last_command = Some(command.clone());
            if let Some(error) = state.accept_error.clone() {
                return Err(error);
            }
            if let Some(snapshot) = state.operations.get(&command.operation_key) {
                return Ok(snapshot.clone());
            }
            let snapshot = CloudComputeK8sOperationSnapshot {
                receipt: CloudComputeK8sAcceptedCreateIntent {
                    request_contract: CloudComputeK8sAcceptanceContract::PendingIntent,
                    operation_key: command.operation_key.clone(),
                    intent: command.intent,
                    request_id: command.request_id,
                    accepted_at_epoch_seconds: ACCEPTED_AT,
                },
                state: OperationState::Accepted,
            };
            state
                .operations
                .insert(command.operation_key, snapshot.clone());
            Ok(snapshot)
        })
    }

    fn get_create_operation<'a>(
        &'a self,
        query: CloudComputeK8sReadCreateOperationQuery,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sOperationLookup, CloudComputeK8sAcceptanceRepositoryError>,
    > {
        Box::pin(async move {
            let mut state = self.state.lock().unwrap();
            state.read_calls += 1;
            if let Some(error) = state.read_error.clone() {
                return Err(error);
            }
            Ok(state
                .operations
                .get(&query.operation_key)
                .cloned()
                .map(CloudComputeK8sOperationLookup::Found)
                .unwrap_or(CloudComputeK8sOperationLookup::NotObserved))
        })
    }
}
