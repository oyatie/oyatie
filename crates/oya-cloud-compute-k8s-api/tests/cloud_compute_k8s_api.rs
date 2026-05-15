use oya_cloud_compute_domain::{CloudComputeCatalog, CloudComputeError};
use oya_cloud_compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CloudComputeK8sApiAuthorization,
    CloudComputeK8sApiBoundaryContext, CloudComputeK8sApiError, CloudComputeK8sApiPrincipal,
    CloudComputeK8sClusterCreateApiRequest, CloudComputeK8sClusterCreateApiStatus,
    CloudComputeK8sClusterCreateRequest, CloudComputeK8sCreateIdempotencyLedger,
    CloudComputeK8sNodePoolCreateRequest, CloudComputeK8sNodePoolFlavorSpec,
    CloudComputeK8sQuotaEnvelope, CloudComputeK8sSecurityGroupRef,
    create_cloud_compute_k8s_cluster_from_api,
};

const CLUSTER_ID: &str = "oya:cloud:kr-seoul:ten_kr:k8s:prod";

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudComputeK8sApiPrincipal {
    CloudComputeK8sApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudComputeK8sApiAuthorization {
    CloudComputeK8sApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn quota() -> CloudComputeK8sQuotaEnvelope {
    CloudComputeK8sQuotaEnvelope {
        vcpu_limit: 128,
        memory_gb_limit: 512,
        gpu_limit: 8,
        local_ssd_gb_limit: 4_096,
        current_vcpu: 4,
        current_memory_gb: 16,
        current_gpu: 0,
        current_local_ssd_gb: 100,
    }
}

fn node_flavor() -> CloudComputeK8sNodePoolFlavorSpec {
    CloudComputeK8sNodePoolFlavorSpec {
        class: "general_purpose".to_string(),
        vcpu: 4,
        memory_gb: 16,
        gpu_count: 0,
        local_ssd_gb: 100,
    }
}

fn node_pool(id: &str, az: &str, subnet: &str) -> CloudComputeK8sNodePoolCreateRequest {
    CloudComputeK8sNodePoolCreateRequest {
        id: id.to_string(),
        az: az.to_string(),
        cell_id: format!("cell-{az}-001"),
        subnet_id: subnet.to_string(),
        security_groups: vec![
            CloudComputeK8sSecurityGroupRef {
                value: format!("sg_{id}_web"),
                tenant_id: "ten_kr".to_string(),
                region: "kr-seoul".to_string(),
                subnet_id: subnet.to_string(),
            },
            CloudComputeK8sSecurityGroupRef {
                value: format!("sg_{id}_app"),
                tenant_id: "ten_kr".to_string(),
                region: "kr-seoul".to_string(),
                subnet_id: subnet.to_string(),
            },
        ],
        flavor: node_flavor(),
        min_nodes: 1,
        max_nodes: 5,
        autoscaling_enabled: true,
    }
}

fn body(resource_id: &str) -> CloudComputeK8sClusterCreateRequest {
    CloudComputeK8sClusterCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        flavor: "high_availability".to_string(),
        control_plane_version: "v1.30.2-oya.1".to_string(),
        control_plane_private: true,
        node_pools: vec![
            node_pool(
                "np_a",
                "kr-seoul-a",
                "oya:cloud:kr-seoul:ten_kr:subnet:prod-a",
            ),
            node_pool(
                "np_b",
                "kr-seoul-b",
                "oya:cloud:kr-seoul:ten_kr:subnet:prod-b",
            ),
            node_pool(
                "np_c",
                "kr-seoul-c",
                "oya:cloud:kr-seoul:ten_kr:subnet:prod-c",
            ),
        ],
        quota: quota(),
        residency: "strict_kr".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_100_010,
    }
}

fn request(request_id: &str, idempotency_key: &str) -> CloudComputeK8sClusterCreateApiRequest {
    CloudComputeK8sClusterCreateApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: authorization_for("sp_compute", &[CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE]),
        body: body(CLUSTER_ID),
    }
}

#[test]
fn api_surface_status_contracts_are_covered() {
    assert_eq!(
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
        "cloud.compute.k8s.cluster.create"
    );
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Created.code(), 201);
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeK8sClusterCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeK8sClusterCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn k8s_create_api_creates_cluster_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request("req-compute-k8s-create", "idem-compute-k8s-create");

    let first =
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("authorized cluster create succeeds");
    let second = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-k8s-create");
    assert_eq!(first.data.resource_id, CLUSTER_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.flavor, "high_availability");
    assert_eq!(first.data.control_plane_version, "v1.30.2-oya.1");
    assert!(first.data.control_plane_private);
    assert_eq!(first.data.node_pool_count, 3);
    assert_eq!(first.data.residency, "strict_kr");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn k8s_create_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-drift", "idem-compute-k8s-drift");
    request.body.resource_id = "oya:cloud:kr-seoul:ten_kr:k8s:other".to_string();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body cluster drift is rejected");

    assert_eq!(
        error,
        CloudComputeK8sApiError::ClusterIdMismatch {
            path_cluster_id: CLUSTER_ID.to_string(),
            body_resource_id: "oya:cloud:kr-seoul:ten_kr:k8s:other".to_string(),
        }
    );
    assert_eq!(error.cluster_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_rejects_unauthorized_same_tenant_principal_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-authz", "idem-compute-k8s-authz");
    request.authorization.allowed_surfaces = vec!["cloud.compute.vm.create".to_string()];

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("authorization decision does not allow cluster create");

    assert_eq!(
        error,
        CloudComputeK8sApiError::AuthorizationDenied {
            surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.cluster_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-authn", "idem-compute-k8s-authn");
    request.principal.principal_id.clear();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(error, CloudComputeK8sApiError::EmptyPrincipalId);
    assert_eq!(error.cluster_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_replays_with_refreshed_authz_and_reordered_pools() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-k8s-authz-refresh-1",
        "idem-compute-k8s-authz-refresh",
    );
    let first =
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("initial cluster create succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-k8s-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.vm.create".to_string(),
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
    ];
    retry.body.node_pools.reverse();
    for pool in &mut retry.body.node_pools {
        pool.security_groups.reverse();
    }
    let second = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_rejects_foreign_security_group_proof_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-sg-proof", "idem-compute-k8s-sg-proof");
    request.body.node_pools[0].security_groups[0].tenant_id = "ten_other".to_string();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("security group proof must match tenant boundary");

    assert!(matches!(
        error,
        CloudComputeK8sApiError::NodePoolSecurityGroupBindingMismatch { .. }
    ));
    assert_eq!(error.cluster_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request("req-compute-k8s-idem", "idem-compute-k8s-idem");
    create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.control_plane_version = "v1.31.0-oya.1".to_string();
    assert_eq!(
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeK8sApiError::IdempotencyKeyReused {
            idempotency_key: "idem-compute-k8s-idem".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_maps_duplicate_cluster_to_conflict() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-k8s-duplicate-a",
            "idem-compute-k8s-duplicate-a",
        ),
    )
    .expect("first cluster create succeeds");

    let error = create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-k8s-duplicate-b",
            "idem-compute-k8s-duplicate-b",
        ),
    )
    .expect_err("second cluster with same id conflicts");

    assert_eq!(
        error,
        CloudComputeK8sApiError::Compute(CloudComputeError::DuplicateKubernetesCluster)
    );
    assert_eq!(error.cluster_create_status_code(), 409);
    assert_eq!(ledger.len(), 2);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_maps_invalid_cluster_shape_to_bad_request() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-shape", "idem-compute-k8s-shape");
    request.body.node_pools.truncate(1);

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("HA cluster requires three AZs");

    assert_eq!(
        error,
        CloudComputeK8sApiError::Compute(CloudComputeError::KubernetesHaRequiresThreeAzs)
    );
    assert_eq!(error.cluster_create_status_code(), 400);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}
