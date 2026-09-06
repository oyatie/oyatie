use compute_k8s_api::*;

use crate::support::{cluster_id, create_command};

pub(super) fn pending_request(id: &str, key: &str) -> CloudComputeK8sCreateAcceptanceApiRequest {
    let legacy = create_command("ten_alpha", "pending", key).desired_spec;
    CloudComputeK8sCreateAcceptanceApiRequest {
        path_cluster_id: cluster_id("ten_alpha", "pending"),
        boundary: CloudComputeK8sApiBoundaryContext {
            request_id: id.into(),
            tenant_id: "ten_alpha".into(),
            idempotency_key: key.into(),
        },
        principal: CloudComputeK8sApiPrincipal {
            tenant_id: "ten_alpha".into(),
            principal_id: "sp-compute-live".into(),
        },
        authorization: authorization("test-create-proof"),
        body: CloudComputeK8sClusterCreateIntent {
            resource_id: legacy.resource_id,
            tenant_id: legacy.tenant_id,
            region: legacy.region,
            flavor: legacy.flavor,
            control_plane_version: legacy.control_plane_version,
            control_plane_private: legacy.control_plane_private,
            node_pools: legacy
                .node_pools
                .into_iter()
                .map(|pool| CloudComputeK8sNodePoolIntent {
                    id: pool.id,
                    az: pool.az,
                    cell_id: pool.cell_id,
                    subnet_id: pool.subnet_id,
                    security_groups: pool
                        .security_groups
                        .into_iter()
                        .map(|group| group.value)
                        .collect(),
                    flavor: pool.flavor,
                    min_nodes: pool.min_nodes,
                    max_nodes: pool.max_nodes,
                    autoscaling_enabled: pool.autoscaling_enabled,
                })
                .collect(),
            residency: legacy.residency,
            data_class: legacy.data_class,
        },
    }
}

fn authorization(decision: &str) -> CloudComputeK8sApiAuthorization {
    CloudComputeK8sApiAuthorization {
        tenant_id: "ignored-caller-claim".into(),
        principal_id: "ignored-caller-claim".into(),
        decision_id: decision.into(),
        allowed_surfaces: Vec::new(),
        proof: None,
    }
}

pub(super) fn operation_read_request(key: &str) -> CloudComputeK8sOperationReadApiRequest {
    let create = pending_request("read-request", key);
    CloudComputeK8sOperationReadApiRequest {
        path_cluster_id: create.path_cluster_id,
        boundary: create.boundary,
        principal: create.principal,
        authorization: authorization("test-read-proof"),
    }
}

pub(super) fn verifier(
    principal: &CloudComputeK8sApiPrincipal,
    decision: &str,
    surface: &str,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(1_700_100_000).with_authorization_proof(
        CloudComputeK8sApiAuthorizationProof {
            tenant_id: principal.tenant_id.clone(),
            principal_id: principal.principal_id.clone(),
            surface: surface.into(),
            decision_id: decision.into(),
            verified: true,
            issued_at_epoch_seconds: 1_700_099_999,
            expires_at_epoch_seconds: 1_700_100_001,
        },
    )
}

pub(super) async fn accept(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sCreateAcceptanceApiRequest,
) -> Result<CloudComputeK8sCreateAcceptanceResponse, CloudComputeK8sAcceptanceApiError> {
    let proof = verifier(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
    );
    accept_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository, request, &proof,
    )
    .await
}

pub(super) async fn read(
    repository: &impl CloudComputeK8sAcceptanceRepository,
    request: CloudComputeK8sOperationReadApiRequest,
) -> Result<CloudComputeK8sOperationLookup, CloudComputeK8sAcceptanceApiError> {
    let proof = verifier(
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE,
    );
    get_cloud_compute_k8s_operation_from_api_with_authorization_verifier(
        repository, request, &proof,
    )
    .await
}
